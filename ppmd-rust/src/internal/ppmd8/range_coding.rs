use std::io::{Read, Write};

use crate::Error;

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct RangeDecoder<R: Read> {
    pub(crate) range: u32,
    pub(crate) code: u32,
    pub(crate) low: u32,
    reader: R,
}

impl<R: Read> RangeDecoder<R> {
    pub(crate) fn new(reader: R) -> crate::Result<Self> {
        let mut encoder = Self {
            range: 0xFFFFFFFF,
            code: 0,
            low: 0,
            reader,
        };

        for _ in 0..4 {
            encoder.code = encoder.code << 8 | encoder.read_byte().map_err(Error::IoError)?;
        }

        if encoder.code == 0xFFFFFFFF {
            return Err(Error::RangeDecoderInitialization);
        }

        Ok(encoder)
    }

    #[inline(always)]
    pub(crate) fn read_byte(&mut self) -> Result<u32, std::io::Error> {
        let mut buffer = [0];
        self.reader.read_exact(&mut buffer)?;
        Ok(buffer[0] as u32)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct RangeEncoder<W: Write> {
    pub(crate) range: u32,
    pub(crate) low: u32,
    writer: W,
}

impl<W: Write> RangeEncoder<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self {
            range: 0xFFFFFFFF,
            low: 0,
            writer,
        }
    }

    #[inline(always)]
    fn encode(&mut self, start: u32, size: u32, total: u32) {
        self.range /= total;
        self.low += start * self.range;
        self.range *= size;
    }

    pub(crate) fn flush(&mut self) -> Result<(), std::io::Error> {
        for _ in 0..4 {
            let byte = (self.low >> 24) as u8;
            self.writer.write_all(&[byte])?;
            self.low <<= 8;
        }
        self.writer.flush()?;
        Ok(())
    }
}
