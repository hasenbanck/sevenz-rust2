use std::io::Write;

use crate::{
    Error, PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER,
    internal::ppmd7::{Ppmd7, RangeEncoder},
};

/// An encoder to encode data using PPMd7 (PPMdH) with the 7z range coder.
pub struct Ppmd7Encoder<W: Write> {
    ppmd: Ppmd7<RangeEncoder<W>>,
}

impl<W: Write> Ppmd7Encoder<W> {
    /// Creates a new [`Ppmd7Encoder`].
    pub fn new(writer: W, order: u32, mem_size: u32) -> crate::Result<Self> {
        if !(PPMD7_MIN_ORDER..=PPMD7_MAX_ORDER).contains(&order)
            || !(PPMD7_MIN_MEM_SIZE..=PPMD7_MAX_MEM_SIZE).contains(&mem_size)
        {
            return Err(Error::InvalidParameter);
        }

        let ppmd = Ppmd7::new_encoder(writer, order, mem_size)?;

        Ok(Self { ppmd })
    }

    fn inner_flush(&mut self) -> Result<(), std::io::Error> {
        self.ppmd.flush_range_encoder()
    }
}

impl<W: Write> Write for Ppmd7Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        self.ppmd.encode_symbols(buf)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner_flush()
    }
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};

    use super::Ppmd7Encoder;
    use crate::Ppmd7Decoder;

    const ORDER: u32 = 8;
    const MEM_SIZE: u32 = 262144;

    #[test]
    fn ppmd7encoder_encode_decode() {
        let test_data = include_str!("../tests/fixtures/apache2.txt");

        let mut writer = Vec::new();
        {
            let mut encoder = Ppmd7Encoder::new(&mut writer, ORDER, MEM_SIZE).unwrap();
            encoder.write_all(test_data.as_bytes()).unwrap();
            encoder.flush().unwrap();
        }

        let mut decoder = Ppmd7Decoder::new(writer.as_slice(), ORDER, MEM_SIZE).unwrap();

        let mut decoded = vec![0; test_data.len()];
        decoder.read_exact(&mut decoded).unwrap();

        assert_eq!(decoded.as_slice(), test_data.as_bytes());

        let decoded_data = String::from_utf8(decoded).unwrap();

        assert_eq!(decoded_data, test_data);
    }
}
