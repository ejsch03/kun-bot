use waitx::Waker;

use super::prelude::*;

pub fn write_wav_header(num_channels: u16, sample_rate: u32, bits_per_sample: u16) -> Vec<u8> {
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;

    let mut h = Vec::with_capacity(44);
    h.extend_from_slice(b"RIFF");
    h.extend_from_slice(&u32::MAX.to_le_bytes()); // unknown size — streaming
    h.extend_from_slice(b"WAVE");
    h.extend_from_slice(b"fmt ");
    h.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    h.extend_from_slice(&3u16.to_le_bytes()); // PCM
    h.extend_from_slice(&num_channels.to_le_bytes());
    h.extend_from_slice(&sample_rate.to_le_bytes());
    h.extend_from_slice(&byte_rate.to_le_bytes());
    h.extend_from_slice(&block_align.to_le_bytes());
    h.extend_from_slice(&bits_per_sample.to_le_bytes());
    h.extend_from_slice(b"data");
    h.extend_from_slice(&u32::MAX.to_le_bytes()); // unknown size — streaming
    h
}

#[derive(Clone)]
pub struct StreamingSink {
    format: AudioFormat,
    buf: Arc<Mutex<VecDeque<u8>>>,
    tx: Waker,
}

impl StreamingSink {
    pub fn new(format: AudioFormat, buf: Arc<Mutex<VecDeque<u8>>>, tx: Waker) -> Self {
        Self { format, buf, tx }
    }
}

impl Sink for StreamingSink {
    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let bytes = match packet {
            AudioPacket::Samples(samples) => match self.format {
                AudioFormat::F64 => samples.as_bytes().to_vec(),
                AudioFormat::F32 => converter.f64_to_f32(&samples).as_bytes().to_vec(),
                AudioFormat::S32 => converter.f64_to_s32(&samples).as_bytes().to_vec(),
                AudioFormat::S24 => converter.f64_to_s24(&samples).as_bytes().to_vec(),
                AudioFormat::S24_3 => converter.f64_to_s24_3(&samples).as_bytes().to_vec(),
                AudioFormat::S16 => converter.f64_to_s16(&samples).as_bytes().to_vec(),
            },
            AudioPacket::Raw(bytes) => bytes,
        };
        // let n = bytes.len();
        self.buf.lock().extend(bytes);
        self.tx.signal();
        // log::debug!("SIGNAL: {}", n);
        Ok(())
    }
}
