use std::io::Read;
#[cfg(feature = "compress")]
use std::io::Write;

const MAX_DISTANCE: usize = 256;
const _MIN_DISTANCE: usize = 1;
const DIS_MASK: usize = MAX_DISTANCE - 1;

struct Delta {
    distance: usize,
    history: [u8; MAX_DISTANCE],
    pos: u8,
}

impl Delta {
    pub fn new(distance: usize) -> Self {
        Self {
            distance,
            history: [0; MAX_DISTANCE],
            pos: 0,
        }
    }

    pub fn decode(&mut self, buf: &mut [u8]) {
        for item in buf {
            let pos = self.pos as usize;
            let h = self.history[(self.distance.wrapping_add(pos)) & DIS_MASK];
            *item = item.wrapping_add(h);
            self.history[pos & DIS_MASK] = *item;
            self.pos = self.pos.wrapping_sub(1);
        }
    }

    #[cfg(feature = "compress")]
    pub fn encode(&mut self, buf: &mut [u8]) {
        for item in buf {
            let pos = self.pos as usize;
            let h = self.history[(self.distance.wrapping_add(pos)) & DIS_MASK];
            let original = *item;
            *item = item.wrapping_sub(h);
            self.history[pos & DIS_MASK] = original;
            self.pos = self.pos.wrapping_sub(1);
        }
    }
}

pub struct DeltaReader<R> {
    inner: R,
    delta: Delta,
}

impl<R> DeltaReader<R> {
    pub fn new(inner: R, distance: usize) -> Self {
        Self {
            inner,
            delta: Delta::new(distance),
        }
    }
}

impl<R: Read> Read for DeltaReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n == 0 {
            return Ok(n);
        }
        self.delta.decode(&mut buf[..n]);
        Ok(n)
    }
}

#[cfg(feature = "compress")]
pub struct DeltaWriter<W> {
    inner: W,
    delta: Delta,
    buffer: Vec<u8>,
}

#[cfg(feature = "compress")]
impl<W> DeltaWriter<W> {
    pub fn new(inner: W, distance: usize) -> Self {
        Self {
            inner,
            delta: Delta::new(distance),
            buffer: Vec::with_capacity(4096),
        }
    }
}

#[cfg(feature = "compress")]
impl<W: Write> Write for DeltaWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let data_size = buf.len();

        if data_size > self.buffer.len() {
            self.buffer.resize(data_size, 0);
        }

        self.buffer[..data_size].copy_from_slice(buf);
        self.delta.encode(&mut self.buffer[..data_size]);
        self.inner.write(&self.buffer[..data_size])
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[cfg(feature = "compress")]
    #[test]
    fn test_delta_roundtrip() {
        let test_cases = [
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            vec![1, 2, 3, 1, 2, 3, 1, 2, 3],
            vec![42, 13, 255, 0, 128, 64, 32, 99, 200, 150],
            vec![100; 20],
            vec![0, 255, 0, 255, 0, 255, 0, 255],
            (0..300).map(|i| (i % 256) as u8).collect(),
        ];

        let distances = vec![1, 2, 4, 8, 16, 32, 64, 128, 256];

        for distance in distances {
            for (i, original_data) in test_cases.iter().enumerate() {
                let mut encoded_buffer = Vec::new();
                let mut writer = DeltaWriter::new(Cursor::new(&mut encoded_buffer), distance);
                std::io::copy(&mut original_data.as_slice(), &mut writer)
                    .expect("Failed to encode data");

                let mut decoded_data = Vec::new();
                let mut reader = DeltaReader::new(Cursor::new(&encoded_buffer), distance);
                std::io::copy(&mut reader, &mut decoded_data).expect("Failed to decode data");

                assert_eq!(
                    original_data, &decoded_data,
                    "Roundtrip failed for distance {} with data set {}",
                    distance, i
                );
            }
        }
    }
}
