use std::io::Read;

use crate::{
    Error, PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER, SYM_END,
    internal::ppmd7::{Ppmd7, RangeDecoder},
};

/// A decoder to decompress data using PPMd7 (PPMdH) with the 7z range coder.
pub struct Ppmd7Decoder<R: Read> {
    ppmd: Ppmd7<RangeDecoder<R>>,
    finished: bool,
}

impl<R: Read> Ppmd7Decoder<R> {
    /// Creates a new [`Ppmd7Decoder`] which provides a reader over the uncompressed data.
    ///
    /// The given `order` must be between [`PPMD7_MIN_ORDER`] and [`PPMD7_MAX_ORDER`]
    /// The given `mem_size` must be between [`PPMD7_MIN_MEM_SIZE`] and [`PPMD7_MAX_MEM_SIZE`]
    pub fn new(reader: R, order: u32, mem_size: u32) -> crate::Result<Self> {
        if !(PPMD7_MIN_ORDER..=PPMD7_MAX_ORDER).contains(&order)
            || !(PPMD7_MIN_MEM_SIZE..=PPMD7_MAX_MEM_SIZE).contains(&mem_size)
        {
            return Err(Error::InvalidParameter);
        }

        let ppmd = Ppmd7::new_decoder(reader, order, mem_size)?;

        Ok(Self {
            ppmd,
            finished: false,
        })
    }

    /// Returns the inner reader.
    pub fn into_inner(self) -> R {
        self.ppmd.into_inner()
    }
}

impl<R: Read> Read for Ppmd7Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.finished {
            return Ok(0);
        }

        if buf.is_empty() {
            return Ok(0);
        }

        let mut sym = 0;
        let mut decoded = 0;

        for byte in buf.iter_mut() {
            match self.ppmd.decode_symbol() {
                Ok(symbol) => sym = symbol,
                Err(err) => {
                    self.finished = true;
                    return Err(err);
                }
            }

            if sym < 0 {
                break;
            }

            *byte = sym as u8;
            decoded += 1;
        }

        if sym >= 0 {
            return Ok(decoded);
        }

        self.finished = true;

        let code = self.ppmd.range_decoder_code();

        if sym != SYM_END || code != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Error during PPMd decoding",
            ));
        }

        Ok(decoded)
    }
}
