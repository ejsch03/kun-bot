use super::prelude::*;

pub fn write_wav_header(num_channels: u16, sample_rate: u32, bits_per_sample: u16) -> [u8; 44] {
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;

    let mut h = [0u8; _];
    h[0..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    h[8..12].copy_from_slice(b"WAVE");
    h[12..16].copy_from_slice(b"fmt ");
    h[16..20].copy_from_slice(&16u32.to_le_bytes());
    h[20..22].copy_from_slice(&3u16.to_le_bytes());
    h[22..24].copy_from_slice(&num_channels.to_le_bytes());
    h[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    h[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    h[32..34].copy_from_slice(&block_align.to_le_bytes());
    h[34..36].copy_from_slice(&bits_per_sample.to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    h
}

pub struct StreamingSink {
    format: AudioFormat,
    buf: HeapProd<u8>,
    tx: Waker,
    rx: Waiter,
}

impl StreamingSink {
    pub fn new(format: AudioFormat, buf: HeapProd<u8>, tx: Waker, rx: Waiter) -> Self {
        Self {
            format,
            buf,
            tx,
            rx,
        }
    }
}

impl Sink for StreamingSink {
    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let bytes: Vec<u8> = match packet {
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

        // until all have been written
        let mut remaining = bytes.as_slice();
        while !remaining.is_empty() {
            let n = self.buf.push_slice(remaining);
            remaining = &remaining[n..];
            if !remaining.is_empty() {
                self.rx.wait();
            }
        }

        self.tx.signal();
        Ok(())
    }
}
