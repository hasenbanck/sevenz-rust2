use std::io::Read;

use crc32fast::Hasher;

pub struct SourceReader<R> {
    reader: R,
    size: usize,
    crc: Hasher,
    crc_value: u32,
}

impl<R> From<R> for SourceReader<R> {
    fn from(value: R) -> Self {
        Self::new(value)
    }
}

impl<R: Read> Read for SourceReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.reader.read(buf)?;
        if self.crc_value == 0 {
            if n > 0 {
                self.size += n;
                self.crc.update(&buf[..n]);
            } else {
                let crc = std::mem::replace(&mut self.crc, Hasher::new());
                self.crc_value = crc.finalize();
            }
        }
        Ok(n)
    }
}

impl<R> SourceReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            size: 0,
            crc: Hasher::new(),
            crc_value: 0,
        }
    }

    pub fn read_count(&self) -> usize {
        self.size
    }

    pub fn crc_value(&self) -> u32 {
        self.crc_value
    }
}
